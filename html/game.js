'use strict';

let canvas = document.getElementById("canvas");
let ctx = canvas.getContext("2d");
canvas.width = 600;
canvas.height = 600;

let qcArr = [];

let ws = new WebSocket("ws://" + location.host + ":3012");
ws.binaryType = "arraybuffer";

let worldData = new Uint8Array(128*128);
let player = {
	x: 0, // float
	y: 0
}

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

function step(sc) {
	player.x += .1;
	player.y += .2;
	clearWcol((player.x & 127) ^ 64);
	clearWrow((player.y & 127) ^ 64);
	if (sc % 32 == 0) {
		qcGetTiles(10);
		qcSend();
	}
}

function draw() { // i don't know what these stand for, even though i just made them
	let ts = 8; // does what the acronym stand for matter
	let xba = canvas.width / ts / 2; // what truly matters in life?
	let yba = canvas.height / ts / 2; // not the acronym
	// but ts is tile width/height in pixels
	// so p.x +- cw / ts / 2 are tile coordinates on edges of canvas
	for (let dy = Math.floor(player.y - yba); dy < player.y + yba; ++dy)
	for (let dx = Math.floor(player.x - xba); dx < player.x + xba; ++dx) {
		let tile = getTile(Math.floor(dx), Math.floor(dy));
		let cx = (dx - player.x) * ts + canvas.width / 2;
		let cy = (dy - player.y) * ts + canvas.height / 2;
		ctx.beginPath();
		ctx.fillStyle = tile == 0 ? "#000" : tile == 0x80 ? "#888480" : tile == 0x81 ? "#444" : "#ff0";
		ctx.fillRect(Math.floor(cx), Math.floor(cy), ts, ts);
		ctx.closePath();
	}
	ctx.beginPath();
	ctx.arc(canvas.width / 2, canvas.height / 2, ts / 3, 0, 2 * Math.PI);
	ctx.fillStyle = "#f0f";
	ctx.fill();
	ctx.closePath();
}
draw();

let sc = 0;

function uaToInt(ua, i) {
	return (ua[i] >> 7 << 31) | ((ua[i] & 127) << 24) | (ua[i+1] << 16) | (ua[i+2] << 8) | ua[i+3];
}
function intToArr(arr, x) {
	arr.push(((x < 0) << 7) | ((x >> 23) & 127));
	arr.push((x >> 16) & 255);
	arr.push((x >> 8) & 255);
	arr.push(x & 255);
}

function qcGetTiles(r) {
	qcArr.push(0);
	intToArr(qcArr, player.x);
	intToArr(qcArr, player.y);
	qcArr.push(r);
}
function qcSend() {
	if (qcArr.length == 0) return;
	let ab = new ArrayBuffer(qcArr.length);
	let ua = new Uint8Array(ab);
	for (let i = 0; i < qcArr.length; ++i)
		ua[i] = qcArr[i];
	console.log(ua);
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
	console.log(x, y);
	for (let wy = y - r; wy <= y + r; ++wy) {
	for (let wx = x - r; wx <= x + r; ++wx) {
		setTile(wx, wy, ua[i]);
		++i;
	}}
	return i;
}

ws.onmessage = function(e) {
	console.log(e);
	let ua = new Uint8Array(e.data);
	let i = 0;
	while (i != ua.length) {
		let code = ua[i];
		++i;
		switch (code) {
		case 0:
			i = hcSetTiles(ua, i);
			break;
		}
	}
	console.log(ua);
}

ws.onopen = function() {
	let ab = new ArrayBuffer(10);
	let ua = new Uint8Array(ab);
	let d = [0 , 0,0,0,5 , 0,0,0,0 , 10];
	for (let i = 0; i < d.length; ++i)
		ua[i] = d[i];
	ws.send(ab);
	setInterval(function() {
		step(sc);
		draw();
		++sc;
	}, 1000 / 32);
}
