"""Exercise launcher search and exact-address window selection with real input."""

def run(driver):
    driver.wait_controls('openLauncher')
    driver.focus_window()
    driver.click('openLauncher')
    assert driver.request('panel')['panel'] == 'launcher'
    driver.wait_controls('launcherSearch')
    driver.click('launcherSearch')
    driver.xdo('type', '--clearmodifiers', 'browser')
    driver.wait_controls('launcherResult1', False)
    driver.xdo('key', 'Return')
    assert driver.request('launch_desktop')['desktop_id'] == 'org.example.Firefox.desktop'
    assert driver.request('panel')['open'] is False
    driver.wait_controls('launcherSearch', False)
    driver.click('openLauncher')
    driver.request('panel')
    driver.wait_controls('launcherWindows')
    driver.click('launcherWindows')
    assert driver.request('panel')['panel'] == 'switcher'
    driver.wait_controls('launcherResult1')
    driver.click('launcherSearch')
    driver.xdo('key', 'ctrl+a', 'BackSpace', 'Down', 'Return')
    assert driver.request('focus_window')['address'] == '0xb'
    assert driver.request('panel')['open'] is False
    print('LAUNCHER INTERACTION PASS: keyword search, Enter, tabs and exact-address switching')
