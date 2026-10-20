# Controller service journal — redacted derivative

The original query covered NetworkManager and wpa_supplicant from the start of
the authorized controller through Wi-Fi restoration (exit 0, empty stderr).
It returned these 18 wpa_supplicant lines; no NetworkManager entries appeared.
The probe-only journal query returned no entries. This describes available
journal output, not a guarantee of complete service logging.

Redactions replace the laptop hostname, MAC addresses and the previous Wi-Fi
SSID with `<laptop>`, `<MAC>` and `<prior-wifi>`. All lines and timestamps are
retained. The unchanged raw controller journal remains local at
`/tmp/rovi-wifi-soak-2026-10-20/controller-journal.json` (private directory),
not committed. No raw file was edited.

```text
2026-10-20T13:19:30.061876+02:00 <laptop> wpa_supplicant[2016]: nl80211: send_event_marker failed: Source based routing not supported
2026-10-20T13:19:30.061895+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-DISCONNECTED bssid=<MAC> reason=3 locally_generated=1
2026-10-20T13:19:30.074991+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-DSCP-POLICY clear_all
2026-10-20T13:19:33.135947+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: SME: Trying to authenticate with <MAC> (SSID='Rovi M8' freq=2412 MHz)
2026-10-20T13:19:33.179851+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: Trying to associate with <MAC> (SSID='Rovi M8' freq=2412 MHz)
2026-10-20T13:19:33.189763+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: Associated with <MAC>
2026-10-20T13:19:33.190047+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-SUBNET-STATUS-UPDATE status=0
2026-10-20T13:19:33.221848+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: WPA: Key negotiation completed with <MAC> [PTK=CCMP GTK=CCMP]
2026-10-20T13:19:33.221880+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-CONNECTED - Connection to <MAC> completed [id=0 id_str=]
2026-10-20T13:34:37.928175+02:00 <laptop> wpa_supplicant[2016]: nl80211: send_event_marker failed: Source based routing not supported
2026-10-20T13:34:37.928210+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-DISCONNECTED bssid=<MAC> reason=3 locally_generated=1
2026-10-20T13:34:37.933983+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-DSCP-POLICY clear_all
2026-10-20T13:34:40.824209+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: SME: Trying to authenticate with <MAC> (SSID='<prior-wifi>' freq=5220 MHz)
2026-10-20T13:34:40.871531+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: Trying to associate with <MAC> (SSID='<prior-wifi>' freq=5220 MHz)
2026-10-20T13:34:40.888038+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: Associated with <MAC>
2026-10-20T13:34:40.888096+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-SUBNET-STATUS-UPDATE status=0
2026-10-20T13:34:40.942024+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: WPA: Key negotiation completed with <MAC> [PTK=CCMP GTK=CCMP]
2026-10-20T13:34:40.942062+02:00 <laptop> wpa_supplicant[2016]: wlp1s0f0: CTRL-EVENT-CONNECTED - Connection to <MAC> completed [id=0 id_str=]
```
