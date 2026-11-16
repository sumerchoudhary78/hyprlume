import QtQuick

// Perspective-projected terrain. Paint only on geometry changes: the dense
// mesh creates depth without a permanent 3D render loop behind the console.
Canvas {
    id: root
    onWidthChanged: requestPaint()
    onHeightChanged: requestPaint()
    onPaint: {
        const c = getContext("2d"), w = width, h = height;
        if (w <= 0 || h <= 0) return;
        c.clearRect(0, 0, w, h);
        const haze = c.createRadialGradient(w*0.55,h*0.75,0,w*0.55,h*0.75,w*0.55);
        haze.addColorStop(0,"rgba(61,80,177,0.30)"); haze.addColorStop(1,"rgba(1,8,23,0)");
        c.fillStyle = haze; c.fillRect(0,0,w,h);
        const r = Math.min(w*0.23,h*0.79), cx=w*0.57, cy=h*0.99;
        const halo = c.createLinearGradient(cx-r,0,cx+r,h);
        halo.addColorStop(0,"#ffbb7d");halo.addColorStop(0.45,"#c48eff");halo.addColorStop(1,"#54d8ff");
        c.beginPath(); c.arc(cx,cy,r,Math.PI*1.02,Math.PI*1.98);
        c.strokeStyle=halo;
        for (const pass of [[14,0.025],[7,0.07],[3,0.24],[1.2,0.95]]) {
            c.lineWidth=pass[0];c.globalAlpha=pass[1];c.stroke();
        }
        c.globalAlpha=1;
        const cols=98, rows=30, grid=[];
        for(let z=0;z<rows;z++) {
            const depth=z/(rows-1), row=[];
            for(let i=0;i<cols;i++) {
                const u=i/(cols-1), x=(u-0.5)*2;
                const peaks = Math.exp(-Math.pow((x+0.73)*5,2))*0.40
                    +Math.exp(-Math.pow((x-0.13)*5,2))*0.92
                    +Math.exp(-Math.pow((x+0.18)*6,2))*0.42;
                const wave=(Math.sin(i*0.61+z*0.39)*0.5+Math.sin(i*1.31-z*0.51)*0.24)*0.10;
                const elevation=(peaks+wave)*Math.pow(1-depth,1.7);
                row.push([w*0.5+x*w*(0.44+depth*0.26),h*(0.58+depth*0.48-elevation*0.45)]);
            }
            grid.push(row);
        }
        const paint = c.createLinearGradient(0,0,w,0);
        paint.addColorStop(0,"#254d97");paint.addColorStop(0.4,"#8864e7");
        paint.addColorStop(0.64,"#6aaaff");paint.addColorStop(1,"#36d5ed");
        c.strokeStyle=paint; c.lineWidth=0.65;
        for(let z=0;z<rows;z++) {
            c.globalAlpha=0.90-z/rows*0.48;
            c.beginPath();grid[z].forEach((p,i)=>i?c.lineTo(p[0],p[1]):c.moveTo(p[0],p[1]));c.stroke();
        }
        c.globalAlpha=0.40;
        for(let i=0;i<cols;i++) {
            c.beginPath();grid.forEach((row,z)=>z?c.lineTo(row[i][0],row[i][1]):c.moveTo(row[i][0],row[i][1]));c.stroke();
        }
        for(let z=0;z<rows;z+=2) for(let i=0;i<cols;i+=2) {
            c.globalAlpha=0.35+(i%7)*0.07;c.fillStyle=i<cols*0.45?"#ac8bff":"#8ce9ff";
            const p=grid[z][i];c.fillRect(p[0],p[1],1.2,1.2);
        }
        c.globalAlpha=1;
    }
}
