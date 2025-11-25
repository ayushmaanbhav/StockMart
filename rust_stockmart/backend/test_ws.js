const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:3000/ws');

ws.on('open', function open() {
    console.log('Connected');

    // 1. Authenticate
    const authMsg = {
        type: 'Auth',
        payload: { token: '1' }
    };
    ws.send(JSON.stringify(authMsg));
});

ws.on('message', function message(data) {
    console.log('Received: %s', data);

    const msg = JSON.parse(data);

    if (msg.type === 'AuthSuccess') {
        console.log('Authenticated!');
        // 2. Place Order
        const orderMsg = {
            type: 'PlaceOrder',
            payload: {
                symbol: 'AAPL',
                side: 'Buy',
                order_type: 'Limit',
                qty: 10,
                price: 1500000 // 150.0000
            }
        };
        ws.send(JSON.stringify(orderMsg));
    } else if (msg.type === 'OrderAck') {
        console.log('Order Placed Successfully!');
        ws.close();
    }
});
