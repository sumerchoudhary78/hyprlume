"""Power input checks use synthetic state; no real power service is loaded."""


def run(driver, mode, height):
    driver.wait_controls('openPower')
    driver.focus_window()
    driver.click('openPower')
    assert driver.request('panel') == {'op': 'panel', 'panel': 'power', 'open': True}
    driver.wait_controls('powerActionsuspend')

    def reveal(name):
        for _ in range(8):
            y = driver.controls[name]['y']
            if 155 < y < height - 65:
                return
            area = driver.controls['powerScroll']
            driver.xdo('mousemove', '--sync', round(area['x']), round(area['y']), 'click', '--repeat', '3', '5' if y >= height - 65 else '4')
            previous, stable = None, 0

            def settled(event):
                nonlocal previous, stable
                if event['kind'] != 'controls':
                    return False
                current = event['controls'][name]['y']
                stable = stable + 1 if previous is not None and abs(current - previous) < 0.5 else 0
                previous = current
                return stable >= 3

            # Flickable animates wheel input. Click the settled control rather
            # than an intermediate rectangle queued during xdotool's scroll.
            driver.wait_event(settled)
        raise AssertionError('could not reveal '+name)
    if mode in ('unavailable', 'inhibited'):
        for action in ('suspend', 'reboot', 'power_off'):
            assert not driver.controls['powerAction'+action]['enabled']
        assert driver.controls['powerActionlogout']['enabled'] == (mode == 'inhibited')
        driver.xdo('key', 'Escape')
        assert driver.request('panel')['open'] is False
        print('POWER INTERACTION PASS: unavailable or inhibited session controls stay disabled')
        return
    driver.click('powerPresentation')
    assert driver.request('idle')['request']=={'action':'presentation','enabled':True}
    driver.click('powerPresentation')
    assert driver.request('idle')['request']=={'action':'presentation','enabled':False}
    reveal('powerProfilepower-saver')
    driver.click('powerProfilepower-saver')
    assert driver.request('power')['request'] == {'action': 'profile', 'profile': 'power-saver'}
    driver.wait_event(lambda e: e['kind'] == 'controls' and e['controls'].get('powerProfilebalanced', {}).get('enabled', False)
                      and not e['controls'].get('powerProfilepower-saver', {}).get('enabled', True))
    reveal('brightness0')
    driver.click('brightness0')
    request = driver.request('power')['request']
    assert request['action'] == 'brightness' and request['device'] == 'sample-panel:1:2:1000'
    assert 5 <= request['percent'] <= 100
    driver.wait_event(lambda e: e['kind'] == 'power' and not e['state']['busy'] and e['state']['backlights'][0]['percent'] == request['percent'])
    driver.wait_event(lambda e: e['kind'] == 'controls' and e['controls'].get('powerActionsuspend', {}).get('enabled', False))

    def prepare(kind):
        reveal('powerAction'+kind)
        driver.click('powerAction'+kind)
        assert driver.request('power')['request'] == {'action': 'prepare', 'kind': kind}
        state = driver.wait_event(lambda e: e['kind'] == 'power' and e['state']['prompt'] is not None)['state']
        driver.wait_controls('confirmPowerAction')
        return state['prompt']['token']

    token = prepare('suspend')
    driver.click('cancelPowerAction')
    assert driver.request('power')['request'] == {'action': 'cancel', 'token': token}
    driver.wait_controls('confirmPowerAction', False)
    token = prepare('logout')
    driver.click('confirmPowerAction')
    assert driver.request('power')['request'] == {'action': 'confirm', 'token': token}
    driver.wait_controls('confirmPowerAction', False)
    driver.wait_event(lambda e: e['kind'] == 'controls' and e['controls'].get('powerActionreboot', {}).get('enabled', False))
    token = prepare('reboot')
    driver.xdo('key', 'Escape')
    assert driver.request('power')['request'] == {'action': 'cancel', 'token': token}
    driver.wait_controls('confirmPowerAction', False)
    driver.xdo('key', 'Escape')
    assert driver.request('panel') == {'op': 'panel', 'panel': 'power', 'open': False}
    driver.wait_controls('powerActionsuspend', False)
    print('POWER INTERACTION PASS: profiles, bound brightness, separate review, cancellation, confirmation and Escape')
