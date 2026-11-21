.pragma library

// Hyprland client positions use logical desktop coordinates, including negative
// monitor origins. An output's reported mode is in physical, unrotated pixels.
function outputBounds(monitor, screen) {
    if (!monitor) return null;
    if (screen && screen.width > 0 && screen.height > 0)
        return {x: monitor.x, y: monitor.y, w: screen.width, h: screen.height};
    const rotated = Number(monitor.lastIpcObject?.transform || 0) % 2 === 1;
    const scale = monitor.scale;
    if (!(scale > 0 && monitor.width > 0 && monitor.height > 0)) return null;
    return {x: monitor.x, y: monitor.y,
        w: (rotated ? monitor.height : monitor.width) / scale,
        h: (rotated ? monitor.width : monitor.height) / scale};
}

function windowFrame(bounds, at, size) {
    if (!bounds || !at || !size || ![at[0], at[1], size[0], size[1]].every(Number.isFinite)
            || size[0] <= 0 || size[1] <= 0) return null;
    const x = Math.max(0, (at[0] - bounds.x) / bounds.w);
    const y = Math.max(0, (at[1] - bounds.y) / bounds.h);
    const right = Math.min(1, (at[0] + size[0] - bounds.x) / bounds.w);
    const bottom = Math.min(1, (at[1] + size[1] - bounds.y) / bounds.h);
    return right > x && bottom > y ? {x: x, y: y, w: right - x, h: bottom - y} : null;
}
