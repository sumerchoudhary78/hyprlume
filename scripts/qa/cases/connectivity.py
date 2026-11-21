"""Real input against the wireless panel, with a strictly synthetic backend."""


def run(driver):
    driver.open_quick_settings()
    driver.wait_controls('openConnectivity')
    assert driver.controls['openConnectivity']['inside'], "Wi-Fi and Bluetooth button is clipped"
    driver.focus_window()
    driver.click('openConnectivity')
    assert driver.request('connectivity')['request']['action'] == 'toggle'
    driver.wait_controls('wifiScan')
    driver.click('wifiScan')
    assert driver.request('connectivity')['request']['action'] == 'wifi_scan'
    driver.wait_event(lambda event: event['kind'] == 'controls' and not event['controls'].get('wifiRadio', {}).get('enabled', True))
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('wifiScan', {}).get('enabled', False))
    print('PASS scans use a native-service request; busy wireless controls disable')

    driver.click('wifiNetwork1')
    driver.wait_controls('wifiPassword')
    driver.click('wifiPassword')
    driver.xdo('type', '--clearmodifiers', '--delay', '15', 'fixture_only_123')
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('wifiConnect', {}).get('enabled', False))
    driver.click('wifiConnect')
    action = driver.request('connectivity')['request']
    assert action['action'] == 'wifi_connect' and action['network'] == 'wifi-lab', action
    assert action['password'] == '[credential]', 'fixture logs exposed a passphrase'
    driver.wait_controls('wifiPassword', False)
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('bluetoothScan', {}).get('enabled', False))
    print('PASS password entry targets the selected network and never enters the agent transcript or QA logs')

    driver.click('bluetoothScan')
    assert driver.request('connectivity')['request']['action'] == 'bluetooth_scan'
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('bluetoothAction1', {}).get('enabled', False))
    driver.click('bluetoothAction1')
    action = driver.request('connectivity')['request']
    assert action['action'] == 'bluetooth_pair' and action['address'] == 'AA:BB:CC:00:00:02', action
    driver.wait_controls('pairingAccept')
    assert not driver.controls['wifiScan']['enabled']
    driver.click('pairingAccept')
    action = driver.request('connectivity')['request']
    assert action['action'] == 'pair_reply' and action['token'] == 77 and action['accept'] is True, action
    driver.wait_controls('pairingAccept', False)
    print('PASS pairing presents a matching-code prompt and replies with its current token')

    driver.wait_controls('bluetoothForget1')
    driver.click('bluetoothForget1')
    driver.wait_event(lambda event: event['kind'] == 'controls')
    driver.click('bluetoothForget1')
    action = driver.request('connectivity')['request']
    assert action['action'] == 'bluetooth_forget' and action['address'] == 'AA:BB:CC:00:00:02', action
    driver.wait_controls('bluetoothAction1')
    driver.click('bluetoothAction1')
    assert driver.request('connectivity')['request']['action'] == 'bluetooth_pair'
    driver.wait_controls('pairingValue')
    driver.xdo('type', '--clearmodifiers', '--delay', '110', '123456')
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('pairingAccept', {}).get('enabled', False))
    driver.click('pairingAccept')
    action = driver.request('connectivity')['request']
    assert action['token'] == 78 and action['value'] == '[credential]', action
    driver.wait_controls('pairingAccept', False)
    assert 'fixture_only_123' not in ''.join(driver.logs), 'fixture logs exposed a password'
    print('PASS forget is device-bound; PIN input survives unrelated snapshots and stays redacted')

    driver.xdo('key', 'Escape')
    assert driver.request('connectivity')['request']['action'] == 'close'
    driver.wait_controls('wifiScan', False)
    print('CONNECTIVITY INTERACTION PASS: scans, credentials, confirmation, PIN, forget and dismissal')
