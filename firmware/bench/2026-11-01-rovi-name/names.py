import asyncio, json, subprocess, sys
from pathlib import Path
from bleak import BleakClient, BleakScanner
import rovi_protocol as rp
sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from apps.teleop.ble import discover

async def read(peer):
    async with peer as c:
        gap=bytes(await c.read_gatt_char("00002a00-0000-1000-8000-00805f9b34fb")).decode()
        status=rp.status_decode(bytes(await c.read_gatt_char(rp.BLE_UUIDS["status"])))
        network=rp.network_decode(bytes(await c.read_gatt_char(rp.BLE_UUIDS["network"])))
        assert gap=="Rovi",gap
        assert status["state"]=="open",status
        return network

async def switch(command):
    assert command in ("home", "car-network")
    dev=await BleakScanner.find_device_by_name("Rovi",timeout=20)
    assert dev,"Rovi not advertising"
    async with BleakClient(dev) as c:
        status=rp.status_decode(bytes(await c.read_gatt_char(rp.BLE_UUIDS["status"])))
        assert status["state"]=="open",status
        await c.write_gatt_char(rp.BLE_UUIDS["control"],rp.control_encode(command),response=True)
        network=rp.network_decode(bytes(await c.read_gatt_char(rp.BLE_UUIDS["network"])))
        assert network["pending"]==command,network
        print("Disarmed mode change:",command,flush=True)
    await asyncio.sleep(10 if command=="home" else 5)

async def main():
    try:
        await switch("car-network")
        peer=await discover("Rovi")
        network=await read(peer)
        assert network["mode"]=="fallback",network
        print("Laptop BLE discovery, GAP and Network:",json.dumps(network),flush=True)
        rows=subprocess.check_output(["nmcli","-t","-f","SSID","device","wifi","list","--rescan","yes"],text=True).splitlines()
        assert "Rovi" in rows,"Rovi SSID absent in Wi-Fi scan"
        assert "Rovi M8" not in rows,"Old Rovi M8 SSID still visible"
        print("Wi-Fi scan: Rovi present; old Rovi M8 absent",flush=True)
    finally:
        await switch("home")
        peer=await discover("Rovi")
        network=await read(peer)
        assert network["mode"]=="home" and network["pending"]=="none",network
        print("Restored Home:",json.dumps(network),flush=True)
asyncio.run(main())
