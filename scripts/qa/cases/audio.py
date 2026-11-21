"""Real input on the shared audio panel; devices and players are synthetic."""


def run(driver):
    driver.open_quick_settings()
    driver.wait_controls('openAudio')
    driver.focus_window()
    driver.click('openAudio')
    assert driver.request('panel') == {'op': 'panel', 'panel': 'audio', 'open': True}
    driver.wait_controls('outputMute0')
    driver.click('outputMute0')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'mute' and e['value'] is True)
    driver.click('outputVolume0')
    initial = driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'volume')['value']
    driver.xdo('key', 'Right', 'Right', 'Right')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'volume' and abs(e['value'] - initial - 0.03) < 0.001)
    driver.click('outputDefault1')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'default' and e['value'] == 'Nebula headphones')
    driver.wait_event(lambda e: e['kind'] == 'controls' and not e['controls'].get('outputDefault1', {}).get('enabled', True))
    driver.click('mediaToggle')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'toggle' and e['value'] is False)
    driver.click('mediaNext')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'next')
    driver.click('mediaPlayer1')
    driver.wait_event(lambda e: e['kind'] == 'controls' and not e['controls'].get('mediaNext', {}).get('enabled', True))
    assert not driver.controls['mediaPrevious']['enabled']
    driver.click('mediaToggle')
    driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'toggle' and e['value'] is True)
    driver.xdo('key', 'Escape')
    assert driver.request('panel') == {'op': 'panel', 'panel': 'audio', 'open': False}
    driver.wait_controls('outputMute0', False)
    print('AUDIO INTERACTION PASS: mute, keyboard volume, default device, player selection, capabilities and Escape')
