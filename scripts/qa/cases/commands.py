"""Idle desktop, menus, command review and keyboard navigation."""


def tab_to(driver, name):
    for _ in range(18):
        driver.xdo('key', 'Tab')
        event = driver.wait_event(lambda e: e['kind'] == 'controls')
        control = event['controls'].get(name, {})
        if control.get('activeFocus') and control['inside']:
            return
    raise AssertionError(f'Tab did not reveal and focus {name}')


def run(driver):
    driver.wait_controls('quickCommand')
    driver.focus_window()
    for name in ('consoleInput', 'systemDetails', 'workspaceRow0', 'openConnectivity', 'toggleVoice'):
        assert name not in driver.controls, f'{name} leaked onto the idle desktop'
    for name in ('openOverview', 'openLauncher', 'quickCommand', 'openNotifications', 'quickSettings'):
        assert driver.controls[name]['inside'], f'{name} is outside the bar'
    driver.click('quickCommand'); driver.request('toggle'); driver.wait_controls('commandInput')
    driver.xdo('type', '--clearmodifiers', '--delay', '25', 'workspace 2'); driver.xdo('key', 'Return')
    r = driver.request('command')
    assert r['text']=='workspace 2' and r['dry_run'] is True, r
    driver.wait_event(lambda e:e['kind']=='controls' and 'commandInput' in e['controls'] and not e['controls']['commandInput']['enabled'] and 'confirmPlan' not in e['controls'])
    assert not driver.controls['previewCommand']['enabled'] and not driver.controls['runDirect']['enabled'], 'busy controls remained enabled'
    driver.wait_controls('confirmPlan')
    assert 'commandInput' not in driver.controls, 'review exposed a replacement command'
    driver.click('confirmPlan'); r=driver.request('confirm'); assert r['token']==42, r
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    print('PASS keyboard entry previews; busy controls disable; review locks input; confirmation carries the token')
    driver.click('quickCommand'); driver.request('toggle'); driver.wait_controls('commandInput')
    driver.xdo('type', '--clearmodifiers', '--delay', '25', 'volume 30')
    driver.wait_event(lambda e:e['kind']=='controls' and e['controls'].get('runDirect',{}).get('enabled',False))
    driver.click('runDirect'); r=driver.request('command'); assert r['text']=='volume 30' and r['dry_run'] is False,r
    driver.wait_event(lambda e:e['kind']=='controls' and 'commandInput' in e['controls'] and e['controls']['commandInput']['enabled'])
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    print('PASS direct execution requires the explicit Run direct button')
    driver.open_quick_settings()
    driver.click('quickMute')
    assert driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'mute')['value'] is True
    driver.click('quickVolume')
    assert 0.4 < driver.wait_event(lambda e: e['kind'] == 'audio' and e['action'] == 'volume')['value'] < 0.6
    driver.click('quickDnd')
    assert driver.request('notifications')['request'] == {'action': 'dnd', 'enabled': True}
    driver.click('toggleVoice'); r=driver.request('voice'); assert r['mode']=='wake',r
    driver.wait_event(lambda e:e['kind']=='controls')
    driver.click('toggleVoice'); r=driver.request('voice'); assert r['mode']=='off',r
    driver.wait_event(lambda e:e['kind']=='controls')
    print('PASS wake-word control sends wake/off; fixture never opens a microphone')
    driver.xdo('key', 'Escape')
    driver.wait_controls('closeDesktopMenu', False)
    driver.click('openOverview')
    driver.wait_stable_controls('consoleInput')
    driver.click('consoleInput')
    driver.xdo('type', '--clearmodifiers', 'volume 20')
    driver.wait_stable_controls('consolePreview')
    driver.click('consolePreview')
    request = driver.request('command')
    assert request['text'] == 'volume 20' and request['dry_run'] is True, request
    driver.wait_controls('cancelPlan')
    driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    print('PASS the Overview editor submits through the same preview and review flow')
    driver.click('openOverview')
    driver.wait_stable_controls('launcherterminal')
    driver.click('launcherterminal')
    request = driver.request('command')
    assert request['text'] == 'launch terminal' and request['dry_run'] is True, request
    driver.wait_controls('cancelPlan'); driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key', 'Escape'); driver.request('toggle'); driver.wait_controls('commandInput', False)
    print('PASS terminal shortcut previews the configured alias')

    driver.click('openOverview')
    driver.wait_stable_controls('workspaceRow2')
    tab_to(driver, 'workspaceRow2')
    driver.xdo('key', 'space')
    event = driver.wait_event(lambda e: e['kind'] == 'workspace')
    assert event['name'] == 'Operations', event
    driver.wait_controls('workspaceRow0', False)
    driver.click('openOverview')
    driver.wait_stable_controls('systemDetails')
    tab_to(driver, 'systemDetails')
    driver.xdo('key', 'space')
    driver.wait_controls('systemDetailsContent')
    driver.xdo('key', 'space')
    driver.wait_controls('systemDetailsContent', False)
    driver.xdo('key', 'Escape')
    driver.wait_controls('closeDesktopMenu', False)
    print('PASS keyboard workspace selection, system disclosure and Escape')
    print('INTERACTION PASS: desktop menus and confirmation flow')
