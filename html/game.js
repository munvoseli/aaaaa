'use strict';

let canvas = document.getElementById("canvas");
let ctx = canvas.getContext("2d");
canvas.width = 600;
canvas.height = 600;

let spirk = document.createElement("canvas");
let spctx = spirk.getContext("2d");

let qcArr = [];
let entities = [];
let radPlayer = 1/2;
let radOrb = 1/3;

let ws = new WebSocket("ws://" + location.host + ":3012");
ws.binaryType = "arraybuffer";

let worldData = new Uint8Array(128*128);
let player = {
	x: 0, // float
	y: 0,
	vx: 0,
	vy: 0,
	drag: 1/64,
	control: 1
}

let controls = {
	dirn: 0,
	dire: 0,
	dirs: 0,
	dirw: 0,
	a: 0,
	b: 0
};

function getTile(x, y) {
	return worldData[(x & 127) | ((y & 127) << 7)];
}
function setTile(x, y, t) {
	worldData[(x & 127) | ((y & 127) << 7)] = t;
}
function clearWcol(col) {
	for (let i = 0; i < 128; ++i)
		worldData[i * 128 + col] = 0;
}
function clearWrow(row) {
	for (let i = 0; i < 128; ++i)
		worldData[row * 128 + i] = 0;
}
function velChange(x, amount, cap) {
	return x < -cap ? -cap : x < -amount ? x + amount : x > cap ? cap : x > amount ? x - amount : 0;
}

function deWorld(x, y) {
	let r = 5;
	let mind = r;
	for (let dy = Math.floor(y - r); dy <= y + r; ++dy)
	for (let dx = Math.floor(x - r); dx <= x + r; ++dx) {
		let tile = getTile(dx, dy);
		if (tile != 0x80) {
			let hx = Math.abs(dx + .5 - x) - .5;
			let hy = Math.abs(dy + .5 - y) - .5;
			let m = Math.max(hx, hy);
			if (m < mind) mind = m;
		}
	}
	return mind;
}
function nmWorld(x, y) {
	let d = 1/16;
	let dex = deWorld(x + d, y) - deWorld(x - d, y);
	let dey = deWorld(x, y + d) - deWorld(x, y - d);
	let mag = Math.sqrt(dex * dex + dey * dey);
	if (mag == 0) return [0, 0];
	return [dex / mag, dey / mag];
}

function movePlayer() {
	let steps = (Math.abs(player.vx) + Math.abs(player.vy)) * 4;
	if (steps < 1) steps = 1;
	if (steps > 30) steps = 16;
	for (let i = 0; i < steps; ++i) {
		let opx = player.x;
		let opy = player.y;
		player.x += player.vx / steps;
		player.y += player.vy / steps;
		let de = deWorld(player.x, player.y) - radPlayer;
		if (de < 0) {
			let nm = nmWorld(player.x, player.y);
			player.x -= nm[0] * de;
			player.y -= nm[1] * de;
			player.vx = (player.x - opx) * steps;
			player.vy = (player.y - opy) * steps;
		}
	}
}

function step(sc) {
	if (player.vx != 0 || player.vy != 0) {
		let vmag = Math.sqrt(player.vx ** 2 + player.vy ** 2);
		let change = velChange(vmag, player.drag, 2) / vmag;
		player.vx *= change;
		player.vy *= change;
	}
//	player.vx = velChange(player.vx, player.drag, 2);
//	player.vy = velChange(player.vy, player.drag, 2);
	let cdx = (controls.dire & 1) - (controls.dirw & 1);
	let cdy = (controls.dirs & 1) - (controls.dirn & 1);
	let bofum = [1, 1.4][+(cdx != 0 && cdy != 0)];
	player.vx += cdx / 8 / bofum;
	player.vy += cdy / 8 / bofum;
	movePlayer();
	clearWcol((player.x & 127) ^ 64);
	clearWrow((player.y & 127) ^ 64);
	if (controls.b == 1) {
		qcBreak(player.x + cdx, player.y + cdy);
	}
	if (sc % 4 == 0) {
		qcSetloc();
		qcGetEntities();
		qcGetTiles(10 + Math.floor(10 * Math.max(Math.abs(player.vx), Math.abs(player.vy))));
	}
	qcSend();
	for (let key in controls) {
		if (controls[key] < 254)
			controls[key] += 2;
	}
}

function draw() { // i don't know what these stand for, even though i just made them
	let ts = 8; // does what the acronym stand for matter
	ts = 16 - 2 * Math.sqrt(player.vx ** 2 + player.vy ** 2);
	let xba = canvas.width / ts / 2; // what truly matters in life?
	let yba = canvas.height / ts / 2; // not the acronym
	// but ts is tile width/height in pixels
	// so p.x +- cw / ts / 2 are tile coordinates on edges of canvas
	for (let dy = Math.floor(player.y - yba); dy < player.y + yba; ++dy)
	for (let dx = Math.floor(player.x - xba); dx < player.x + xba; ++dx) {
		let tile = getTile(Math.floor(dx), Math.floor(dy));
		let cx = (dx - player.x) * ts + canvas.width / 2;
		let cy = (dy - player.y) * ts + canvas.height / 2;
		if (tile < 0x82) {
			ctx.beginPath();
			ctx.fillStyle = tile == 0 ? "#000" : tile == 0x80 ? "#888480" : tile == 0x81 ? "#444" : "#ff0";
			ctx.fillRect(Math.floor(cx), Math.floor(cy), Math.floor(cx + ts) - Math.floor(cx), Math.floor(cy + ts) - Math.floor(cy));
			ctx.closePath();
		} else {
			ctx.drawImage(
				spirk, (tile & 15) << 3, (tile >> 4) << 3, 8, 8,
				Math.floor(cx), Math.floor(cy), Math.floor(cx + ts) - Math.floor(cx), Math.floor(cy + ts) - Math.floor(cy)
			);
		}
	}
	for (let entity of entities) {
		ctx.beginPath();
		if (entity.t == 0)
			ctx.fillStyle = "#ff0";
		else if (entity.t == 1) {
			ctx.fillStyle = ["#fff", "#5d97ff", "#facb35"][entity.f];
		} else {
			ctx.fillStyle = "#666";
		}
		ctx.arc((entity.x - player.x) * ts + canvas.width / 2, (entity.y - player.y) * ts + canvas.height / 2, ts * [radPlayer, radOrb][entity.t], 0, 2 * Math.PI);
		ctx.fill();
		ctx.closePath();
	}
	ctx.beginPath();
	ctx.arc(canvas.width / 2, canvas.height / 2, ts * radPlayer, 0, 2 * Math.PI);
	ctx.fillStyle = "#f0f";
	ctx.fill();
	ctx.closePath();
}

let sc = 0;

function uaToInt(ua, i) {
	return (ua[i] >> 7 << 31) | ((ua[i] & 127) << 24) | (ua[i+1] << 16) | (ua[i+2] << 8) | ua[i+3];
}
function intToArr(arr, x) {
	x = Math.floor(x);
	arr.push(((x < 0) << 7) | ((x >> 23) & 127));
	arr.push((x >> 16) & 255);
	arr.push((x >> 8) & 255);
	arr.push(x & 255);
}

function qcGetTiles(r) {
	qcArr.push(0);
	intToArr(qcArr, player.x + player.vx * 8);
	intToArr(qcArr, player.y + player.vy * 8);
	qcArr.push(r);
}
function qcSetloc() {
	qcArr.push(1);
	intToArr(qcArr, player.x);
	intToArr(qcArr, player.y);
	qcArr.push((player.x * 256) & 255);
	qcArr.push((player.y * 256) & 255);
}
function qcBreak(x, y) {
	qcArr.push(2);
	intToArr(qcArr, x);
	intToArr(qcArr, y);
}
function qcGetEntities() {
	qcArr.push(3);
}
function qcPlaceTile(x, y, t) {
	qcArr.push(4);
	intToArr(qcArr, x);
	intToArr(qcArr, y);
	qcArr.push(t);
}
function qcSend() {
	if (qcArr.length == 0) return;
	let ab = new ArrayBuffer(qcArr.length);
	let ua = new Uint8Array(ab);
	for (let i = 0; i < qcArr.length; ++i)
		ua[i] = qcArr[i];
	ws.send(ab);
	qcArr = [];
}

function hcSetTiles(ua, i) {
	let x = uaToInt(ua, i);
	i += 4;
	let y = uaToInt(ua, i);
	i += 4;
	let r = ua[i];
	++i;
	for (let wy = y - r; wy <= y + r; ++wy) {
	for (let wx = x - r; wx <= x + r; ++wx) {
		setTile(wx, wy, ua[i]);
		++i;
	}}
	return i;
}
function hcSetEntities(ua, i) {
	let npl = ua[i]; ++i;
	entities = [];
//	console.log(ua);
	for (let j = 0; j < npl; ++j) {
		let t = ua[i]; i++;
		let x = uaToInt(ua, i); i += 4;
		let y = uaToInt(ua, i); i += 4;
		let subx = ua[i]; i++;
		let suby = ua[i]; i++;
		let e = {x: x + subx / 256, y: y + suby / 256, t: t};
		if (t == 1) {
			e.f = ua[i]; i++;
		}
		entities.push(e);
	}
	return i;
}

ws.onmessage = function(e) {
//	console.log(e);
	let ua = new Uint8Array(e.data);
	let i = 0;
	let codes = [];
	while (i < ua.length) {
		let code = ua[i];
		codes.push(code);
		++i;
		switch (code) {
		case 0:
			i = hcSetTiles(ua, i);
			break;
		case 1:
			i = hcSetEntities(ua, i);
			break;
		}
	}
//	console.log(codes);
}


let tryStartCount = 2;
function tryStart() {
	tryStartCount--;
	if (tryStartCount != 0) return;
	setInterval(function() {
		step(sc);
		draw();
		++sc;
	}, 1000 / 32);
}

ws.onopen = function() {
	tryStart();
	ctx.imageSmoothingEnabled = false;
//	let ab = new ArrayBuffer(10);
//	let ua = new Uint8Array(ab);
//	let d = [0 , 0,0,0,5 , 0,0,0,0 , 10];
//	for (let i = 0; i < d.length; ++i)
//		ua[i] = d[i];
//	ws.send(ab);
};

ws.onerror = function() {
	ctx.textAlign = "center";
	ctx.textBaseline = "middle";
	ctx.font = "20px monospace";
	ctx.fillText("could not connect to ws", canvas.width / 2, canvas.height / 2);
};

let spritesheetimage = new Image();
spritesheetimage.onload = function() {
	spirk.width = spritesheetimage.naturalWidth;
	spirk.height = spritesheetimage.naturalHeight;
	spctx.drawImage(spritesheetimage, 0, 0);
	tryStart();
};
spritesheetimage.src = "sprites.png";

function handleKey(k, b) {
	switch (k) {
	case "a": controls.dirw = b; break;
	case "s": controls.dirs = b; break;
	case "d": controls.dire = b; break;
	case "w":
	case "f": controls.dirn = b; break;
	case "j": controls.a = b; break;
	case "k": controls.b = b; break;
	}
}
addEventListener("keyup", function(e) {
	handleKey(e.key, 0);
}, false);
addEventListener("keydown", function(e) {
	if (e.repeat) return false;
	handleKey(e.key, 1);
}, false);
