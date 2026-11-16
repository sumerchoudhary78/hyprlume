import QtQuick
import qs.theme

// Filled trend line. Repaints only when the sample array changes (once a
// second at most), so it is a cooperative canvas and not a frame timer.
Canvas {
    id: root

    property var values: []
    property color accent: Theme.cyan
    property real lineWidth: Theme.px(2)

    renderStrategy: Canvas.Cooperative
    antialiasing: true

    onValuesChanged: requestPaint()
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()

    onPaint: {
        const ctx = getContext("2d");
        ctx.clearRect(0, 0, width, height);

        const samples = root.values && root.values.length > 1 ? root.values : [0.5, 0.5];
        const pad = root.lineWidth;
        const w = width - pad * 2;
        const h = height - pad * 2;
        if (w <= 0 || h <= 0)
            return;

        let low = samples[0];
        let high = samples[0];
        for (let i = 1; i < samples.length; i++) {
            low = Math.min(low, samples[i]);
            high = Math.max(high, samples[i]);
        }
        const span = Math.max(high - low, 0.0001);

        const x = i => pad + w * (i / (samples.length - 1));
        const y = v => pad + h - h * ((v - low) / span);

        ctx.beginPath();
        ctx.moveTo(x(0), y(samples[0]));
        for (let i = 1; i < samples.length; i++)
            ctx.lineTo(x(i), y(samples[i]));

        ctx.lineWidth = root.lineWidth;
        ctx.lineJoin = "round";
        ctx.lineCap = "round";
        ctx.strokeStyle = root.accent;
        ctx.stroke();

        ctx.lineTo(pad + w, pad + h);
        ctx.lineTo(pad, pad + h);
        ctx.closePath();

        const fill = ctx.createLinearGradient(0, 0, 0, height);
        fill.addColorStop(0, Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.22));
        fill.addColorStop(1, Qt.rgba(root.accent.r, root.accent.g, root.accent.b, 0.0));
        ctx.fillStyle = fill;
        ctx.fill();
    }
}
