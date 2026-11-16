.pragma library

function spherePoint(lon, lat, radius) {
    const a = lon * Math.PI / 180, b = lat * Math.PI / 180;
    return Qt.vector3d(radius * Math.cos(b) * Math.sin(a), radius * Math.sin(b), radius * Math.cos(b) * Math.cos(a));
}
function coasts(rings) {
    const out = [];
    for (const ring of rings) for (let i = 1; i < ring.length; i++)
        out.push(spherePoint(ring[i-1][0],ring[i-1][1],57),spherePoint(ring[i][0],ring[i][1],57));
    return out;
}
function land(points) {
    return points.map(p => spherePoint(p[0], p[1], 57));
}
function grid() {
    const out = [];
    for (let lat=-60; lat<=60; lat+=20) for(let lon=0; lon<360; lon+=5)
        out.push(spherePoint(lon,lat,56),spherePoint(lon+5,lat,56));
    for (let lon=0; lon<360; lon+=20) for(let lat=-90; lat<90; lat+=5)
        out.push(spherePoint(lon,lat,56),spherePoint(lon,lat+5,56));
    return out;
}
function city() {
    const wire=[], solid=[], colors=[];
    function edge(a,b) { wire.push(Qt.vector3d(a[0],a[1],a[2]),Qt.vector3d(b[0],b[1],b[2])); }
    function face(points, corners, color) {
        for(const i of [0,1,2,0,2,3]) {
            const p=points[corners[i]];
            solid.push(Qt.vector3d(p[0],p[1],p[2])); colors.push(Qt.vector4d(color[0],color[1],color[2],1));
        }
    }
    for(let i=0;i<55;i++) {
        const x=(i%11-5)*14,z=(Math.floor(i/11)-2)*15;
        const h=9+(i*37%59)+(i===26?56:0),w=4+i%3;
        const p=[[x-w,0,z-w],[x+w,0,z-w],[x+w,0,z+w],[x-w,0,z+w],
            [x-w,h,z-w],[x+w,h,z-w],[x+w,h,z+w],[x-w,h,z+w]];
        face(p,[0,1,5,4],[0.035,0.19,0.30]);face(p,[1,2,6,5],[0.05,0.29,0.41]);
        face(p,[2,3,7,6],[0.03,0.22,0.34]);face(p,[3,0,4,7],[0.03,0.13,0.23]);
        face(p,[4,5,6,7],[0.08,0.40,0.57]);
        for(const e of [[0,1],[1,2],[2,3],[3,0],[4,5],[5,6],[6,7],[7,4],[0,4],[1,5],[2,6],[3,7]]) edge(p[e[0]],p[e[1]]);
        for(let y=8;y<h;y+=9) {
            edge([x-w,y,z-w],[x+w,y,z-w]);edge([x+w,y,z-w],[x+w,y,z+w]);edge([x+w,y,z+w],[x-w,y,z+w]);
        }
    }
    for(let x=-95;x<=95;x+=19) edge([x,0,-55],[x,0,55]);
    for(let z=-55;z<=55;z+=11) edge([-95,0,z],[95,0,z]);
    return {wire:wire,solid:solid,colors:colors};
}
