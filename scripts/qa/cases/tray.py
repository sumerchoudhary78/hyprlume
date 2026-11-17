"""Production tray controls driven with synthetic item/menu handles."""


def run(driver, mode):
    driver.wait_controls('openTray')
    driver.focus_window()
    driver.click('openTray')
    assert driver.request('panel') == {'op': 'panel', 'panel': 'tray', 'open': True}
    driver.wait_controls('closeTray')
    if mode == 'empty':
        assert 'trayOpen0' not in driver.controls
        driver.xdo('key', 'Escape')
        assert driver.request('panel')['open'] is False
        print('TRAY INTERACTION PASS: empty state and Escape')
        return
    driver.wait_controls('trayMenuEntry4')
    assert not driver.controls['trayMenuEntry4']['enabled']
    driver.click('trayOpen0')
    driver.wait_event(lambda e: e['kind'] == 'tray' and e['action'] == 'activate')
    assert driver.request('panel')['open'] is False
    driver.wait_controls('closeTray', False)
    driver.click('openTray')
    assert driver.request('panel')['open'] is True
    driver.wait_controls('trayMenuEntry4')
    position = driver.controls['trayIcon0']
    driver.xdo('mousemove', '--sync', round(position['x']), round(position['y']), 'click', '2')
    driver.wait_event(lambda e: e['kind'] == 'tray' and e['action'] == 'secondary')
    driver.click('trayMenuEntry2')
    driver.wait_event(lambda e: e['kind'] == 'tray' and e['action'] == 'menu' and e['text'] == 'Show alerts' and e['checked'] is False)
    driver.click('trayMenuEntry3')
    driver.wait_controls('trayBack')
    driver.wait_controls('trayMenuEntry1', False)
    driver.click('trayMenuEntry0')
    driver.wait_event(lambda e: e['kind'] == 'tray' and e['action'] == 'menu' and e['text'] == 'Connection details')
    driver.xdo('key', 'Escape')
    driver.wait_controls('trayBack', False)
    driver.xdo('key', 'Home')
    driver.wait_event(lambda e: e['kind'] == 'controls' and e['controls'].get('trayMenuEntry0', {}).get('activeFocus', False))
    driver.xdo('key', 'Return')
    driver.wait_event(lambda e: e['kind'] == 'tray' and e['action'] == 'menu' and e['text'] == 'Open control centre')
    driver.xdo('key', 'Escape')
    assert driver.request('panel')['open'] is False
    print('TRAY INTERACTION PASS: activation, secondary action, checkbox, submenu, disabled action and keyboard navigation')
