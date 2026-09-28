

## Added openvpn connection to https://portmap.io/mappings so that external clients can access the service on port 8000 via port forwarding

Get your own client config from portmap.io (do not commit it — `*.ovpn` is gitignored) and install it as `/etc/openvpn/client.conf`:

```
sudo cp <your-client>.ovpn /etc/openvpn/client.conf
sudo systemctl enable openvpn@client.service
sudo systemctl daemon-reload
systemctl status openvpn@client.service
```
