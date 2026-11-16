import QtQuick
import qs.theme

// Static procedural backdrop for the stage: horizon glow, ridgelines, and a
// thin halo arc. It is painted once per resize, never per frame.
Canvas {
    id: root

    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
        const ctx = getContext("2d");
        const w = width;
        const h = height;
        if (w <= 0 || h <= 0)
            return;

        ctx.clearRect(0, 0, w, h);

        const sky = ctx.createLinearGradient(0, 0, 0, h);
        sky.addColorStop(0.0, "#0a1524");
        sky.addColorStop(0.55, "#0d2131");
        sky.addColorStop(1.0, "#123044");
        ctx.fillStyle = sky;
        ctx.fillRect(0, 0, w, h);

        const glow = ctx.createRadialGradient(w * 0.52, h * 1.02, 0, w * 0.52, h * 1.02, w * 0.55);
        glow.addColorStop(0.0, "rgba(164, 139, 255, 0.30)");
        glow.addColorStop(0.45, "rgba(95, 216, 255, 0.12)");
        glow.addColorStop(1.0, "rgba(10, 21, 36, 0.0)");
        ctx.fillStyle = glow;
        ctx.fillRect(0, 0, w, h);

        // Halo arc.
        ctx.save();
        ctx.beginPath();
        ctx.arc(w * 0.5, h * 1.05, Math.min(w * 0.42, h * 0.95), Math.PI * 1.12, Math.PI * 1.88);
        ctx.lineWidth = Math.max(1, h * 0.004);
        ctx.strokeStyle = "rgba(255, 190, 130, 0.55)";
        ctx.shadowColor = "rgba(255, 170, 110, 0.55)";
        ctx.shadowBlur = h * 0.06;
        ctx.stroke();
        ctx.restore();

        // Stars: deterministic lattice, no randomness so repaints match.
        ctx.fillStyle = "rgba(190, 225, 255, 0.35)";
        for (let i = 0; i < 90; i++) {
            const x = ((i * 7919) % w);
            const y = ((i * 104729) % Math.max(1, h * 0.55));
            const r = (i % 3 === 0) ? 1.1 : 0.6;
            ctx.fillRect(x, y, r, r);
        }

        const ridge = (base, amplitude, step, color, phase) => {
            ctx.beginPath();
            ctx.moveTo(0, h);
            for (let x = 0; x <= w; x += step) {
                const wave = Math.sin(x * 0.011 + phase) * 0.55
                    + Math.sin(x * 0.0037 + phase * 2.1) * 0.45;
                ctx.lineTo(x, base - wave * amplitude);
            }
            ctx.lineTo(w, h);
            ctx.closePath();
            ctx.fillStyle = color;
            ctx.fill();
        };

        ridge(h * 0.80, h * 0.10, Math.max(2, w / 220), "rgba(13, 30, 48, 0.85)", 1.2);
        ridge(h * 0.88, h * 0.09, Math.max(2, w / 180), "rgba(9, 21, 35, 0.92)", 3.4);
        ridge(h * 0.96, h * 0.07, Math.max(2, w / 150), "rgba(6, 14, 24, 0.96)", 5.1);
    }
}
