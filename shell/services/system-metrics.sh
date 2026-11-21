#!/bin/sh
# One compact metrics line every two seconds on stdout, consumed by
# shell/services/Metrics.qml.
#
# Every field is best-effort. A missing file, GPU vendor, or container runtime
# reports -1 (or is omitted) instead of killing the sampler, so the shell still
# starts on unusual hardware.
#
# Format:
#   S cpu=<pct> mem=<pct> memmb=<used MiB> gpu=<pct|-1> netup=<KiB/s>
#     netdown=<KiB/s> disk=<used bytes|-1> disktotal=<bytes|-1> load=<1min>
#     uptime=<minutes>
#   C count=<running> name=<container> cpu=<pct> mem=<usage>

set -u

user=0
nice=0
system=0
idle=0
iowait=0
irq=0
softirq=0
steal=0

prev_total=0
prev_idle=0
prev_rx=""
prev_tx=""
prev_sample_time=0
rx=0
tx=0
gpu_value=-1
disk_used=-1
disk_total=-1
tick=0

gpu_sysfs=""
for candidate in /sys/class/drm/card*/device/gpu_busy_percent; do
    if [ -r "$candidate" ]; then
        gpu_sysfs="$candidate"
        break
    fi
done

have_nvidia=0
command -v nvidia-smi >/dev/null 2>&1 && have_nvidia=1

runtime=""
if command -v docker >/dev/null 2>&1; then
    runtime=docker
elif command -v podman >/dev/null 2>&1; then
    runtime=podman
fi

while :; do
    tick=$((tick + 1))

    # -- CPU: delta of the aggregate jiffies line ------------------------
    read -r _ user nice system idle iowait irq softirq steal _ < /proc/stat
    total=$((user + nice + system + idle + iowait + irq + softirq + steal))
    idle_all=$((idle + iowait))
    cpu=-1
    if [ "$prev_total" -gt 0 ]; then
        delta_total=$((total - prev_total))
        delta_idle=$((idle_all - prev_idle))
        if [ "$delta_total" -gt 0 ]; then
            cpu=$(((delta_total - delta_idle) * 100 / delta_total))
        fi
    fi
    prev_total=$total
    prev_idle=$idle_all

    # -- Memory ----------------------------------------------------------
    mem_total=0
    mem_avail=0
    while read -r key value _; do
        case "$key" in
            MemTotal:) mem_total=$value ;;
            MemAvailable:) mem_avail=$value ;;
        esac
    done < /proc/meminfo
    mem_pct=0
    mem_used=0
    if [ "$mem_total" -gt 0 ]; then
        mem_pct=$(((mem_total - mem_avail) * 100 / mem_total))
        mem_used=$(((mem_total - mem_avail) / 1024))
    fi

    # -- GPU -------------------------------------------------------------
    if [ -n "$gpu_sysfs" ]; then
        gpu_value=$(cat "$gpu_sysfs" 2>/dev/null) || gpu_value=-1
    elif [ "$have_nvidia" = 1 ] && [ $((tick % 5)) -eq 1 ]; then
        gpu_value=$(timeout 2 nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader,nounits 2>/dev/null | head -n 1)
    fi
    case "$gpu_value" in
        ''|*[!0-9]*) gpu_value=-1 ;;
    esac

    # -- Network throughput ---------------------------------------------
    rx=0
    tx=0
    # Include the monotonic sample time so a slow runtime/GPU query cannot
    # turn a two-second byte delta into an incorrect per-second rate.
    sample=$(awk -v oldrx="$prev_rx" -v oldtx="$prev_tx" -v oldtime="$prev_sample_time" '
        FILENAME == ARGV[1] { now = $1; next }
        FNR > 2 && $1 !~ /^lo:/ { rx += $2; tx += $10 }
        END {
            elapsed = now - oldtime; up = 0; down = 0
            if (oldtime > 0 && elapsed > 0) {
                up = (tx - oldtx) / 1024 / elapsed
                down = (rx - oldrx) / 1024 / elapsed
            }
            printf "%.0f %.0f %.2f %.0f %.0f", rx, tx, now, up > 0 ? up : 0, down > 0 ? down : 0
        }' /proc/uptime /proc/net/dev 2>/dev/null)
    up=0
    down=0
    if [ -n "$sample" ]; then
        read -r rx tx prev_sample_time up down <<RATES
$sample
RATES
    fi
    prev_rx=$rx
    prev_tx=$tx

    # -- Disk: refreshed every 30 s -------------------------------------
    if [ $((tick % 15)) -eq 1 ]; then
        disk=$(df -P -B1 / 2>/dev/null | awk 'NR == 2 { print $3 " " $2 }')
        if [ -n "$disk" ]; then
            disk_used=${disk% *}
            disk_total=${disk#* }
        fi
    fi

    # -- Load and uptime -------------------------------------------------
    load1=0
    read -r load1 _ < /proc/loadavg
    uptime_minutes=0
    read -r uptime_seconds _ < /proc/uptime
    uptime_minutes=$(( ${uptime_seconds%%.*} / 60 ))

    printf 'S cpu=%d mem=%d memmb=%d gpu=%d netup=%d netdown=%d disk=%d disktotal=%d load=%s uptime=%d\n' \
        "$cpu" "$mem_pct" "$mem_used" "$gpu_value" "$up" "$down" \
        "$disk_used" "$disk_total" "$load1" "$uptime_minutes"

    # -- Containers: refreshed every 10 s ---------------------------------
    # Always emits a line so the UI can tell "no runtime" from "runtime, no
    # containers running".
    if [ $((tick % 5)) -eq 1 ]; then
        count=0
        first=""
        cpu_c=0
        mem_c=""
        if [ -n "$runtime" ]; then
            names=$(timeout 2 "$runtime" ps --format '{{.Names}}' 2>/dev/null)
            count=$(printf '%s\n' "$names" | awk 'NF { n++ } END { print n + 0 }')
            if [ "$count" -gt 0 ]; then
                first=$(printf '%s\n' "$names" | head -n 1)
                stats=$(timeout 3 "$runtime" stats --no-stream --format '{{.CPUPerc}} {{.MemUsage}}' "$first" 2>/dev/null | head -n 1)
                cpu_c=$(printf '%s' "$stats" | awk '{ gsub(/%/, "", $1); print $1 + 0 }')
                mem_c=$(printf '%s' "$stats" | awk '{ print $1 }')
                [ -n "$cpu_c" ] || cpu_c=0
            fi
        fi
        printf 'C count=%d name=%s cpu=%s mem=%s runtime=%s\n' \
            "$count" "$first" "$cpu_c" "$mem_c" "$runtime"
    fi

    sleep 2
done
