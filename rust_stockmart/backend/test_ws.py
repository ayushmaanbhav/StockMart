#!/usr/bin/env python3
import asyncio
import websockets
import json

async def test_websocket():
    uri = "ws://localhost:3000/ws"
    
    async with websockets.connect(uri) as websocket:
        print("Connected to WebSocket")
        
        # 1. Authenticate
        auth_msg = {"type": "Auth", "payload": {"token": "1"}}
        await websocket.send(json.dumps(auth_msg))
        print(f"Sent: {auth_msg}")
        
        # Wait for auth response
        response = await websocket.recv()
        print(f"Received: {response}")
        
        # 2. Subscribe to AAPL
        subscribe_msg = {"type": "Subscribe", "payload": {"symbol": "AAPL"}}
        await websocket.send(json.dumps(subscribe_msg))
        print(f"Sent: {subscribe_msg}")
        
        # 3. Place a buy order
        order_msg = {
            "type": "PlaceOrder",
            "payload": {
                "symbol": "AAPL",
                "side": "Buy",
                "order_type": "Limit",
                "qty": 10,
                "price": 1500000  # 150.00 * 10000
            }
        }
        await websocket.send(json.dumps(order_msg))
        print(f"Sent: {order_msg}")
        
        # 4. Listen for messages for 10 seconds
        try:
            for i in range(20):
                response = await asyncio.wait_for(websocket.recv(), timeout=1.0)
                print(f"Received: {response}")
        except asyncio.TimeoutError:
            print("Timeout waiting for message")
        
        print("\nTest completed")

if __name__ == "__main__":
    asyncio.run(test_websocket())
