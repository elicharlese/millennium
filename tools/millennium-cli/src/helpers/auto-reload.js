// taken from https://github.com/thedodd/trunk/blob/5c799dc35f1f1d8f8d3d30c8723cbb761a9b6a08/src/autoreload.js
(function () {
	const url = `ws://${window.location.host}/__millennium-live/ws`;
	const pollInterval = 5000;

	const onClose = () => {
		window.setTimeout(() => {
			// when we successfully reconnect, we'll force a reload (since we presumably lost connection to the server
			// due to it being killed, so it will haverebuilt on restart)
			const ws = new WebSocket(url);
			ws.addEventListener('open', () => window.location.reload());
			ws.addEventListener('close', onClose);
		}, pollInterval);
	};

	const ws = new WebSocket(url);
	ws.addEventListener('message', ({ data }) => {
		const msg = JSON.parse(data);
		if (msg.reload)
			window.location.reload();
	});
	ws.addEventListener('close', onClose);
})();
