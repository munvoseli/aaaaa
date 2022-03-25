let ws = new WebSocket("ws://" + location.host + ":3012");
ws.binaryType = "arraybuffer";

let worldData = new Uint8Array(128*128);

function uaToInt(ua, i) {
	return [1, -1][ua[i] >> 7] * (((ua[0] & 127) << 24) | (ua[1] << 16) | (ua[2] << 8) | ua[3]);
}

function hcSetTiles(ua, i) {
	let x = uaToInt(ua, i);
	i += 4;
	let y = uaToInt(ua, i);
	i += 4;
	let r = ua[i];
	++i;
	for (let wy = y - r; wy <= r; ++wy)
	for (let wx = x - r; wx <= r; ++wx) {
		worldData[(wx & 127) | ((wy & 127) << 7)] = ua[i];
		++i;
	}
	return i;
}

ws.onmessage = function(e) {
	console.log(e);
	let view = new Uint8Array(e.data);
	console.log(view);
}

ws.onopen = function() {
	let ab = new ArrayBuffer(10);
	let ua = new Uint8Array(ab);
	let d = [0 , 0,0,0,0 , 0,0,0,0 , 3];
	for (let i = 0; i < d.length; ++i)
		ua[i] = d[i];
	console.log(ua, ab);
	ws.send(ab);
}
