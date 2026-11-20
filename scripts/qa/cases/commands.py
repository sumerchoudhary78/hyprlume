"""Agent command, review and dashboard controls."""


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
    driver.click('toggleVoice'); r=driver.request('voice'); assert r['mode']=='wake',r
    driver.wait_event(lambda e:e['kind']=='controls')
    driver.click('toggleVoice'); r=driver.request('voice'); assert r['mode']=='off',r
    driver.wait_event(lambda e:e['kind']=='controls')
    print('PASS wake-word control sends wake/off; fixture never opens a microphone')
    driver.click('consoleInput')
    driver.xdo('type', '--clearmodifiers', 'volume 20')
    driver.wait_stable_controls('consolePreview')
    driver.click('consolePreview')
    request = driver.request('command')
    assert request['text'] == 'volume 20' and request['dry_run'] is True, request
    driver.wait_controls('cancelPlan')
    driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    print('PASS the home editor submits through the same preview and review flow')
    driver.click('agentVolume'); r=driver.request('command'); assert r['text']=='volume 35' and r['dry_run'] is True,r
    driver.wait_controls('cancelPlan'); driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    driver.click('launcherterminal'); r=driver.request('command'); assert r['text']=='launch terminal' and r['dry_run'] is True,r
    driver.wait_controls('cancelPlan'); driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    print('PASS shortcuts preview; dismissal cancels; terminal shortcut still uses the allowlisted alias')

    driver.click('consoleInput')
    tab_to(driver, 'workspaceRow2')
    driver.xdo('key', 'space')
    event = driver.wait_event(lambda e: e['kind'] == 'workspace')
    assert event['name'] == 'Operations', event
    for name in ('quickCommand', 'openLauncher', 'openConnectivity', 'openDesktopTools', 'openSettings', 'openPower'):
        assert driver.controls[name]['inside'], f'{name} is outside the window'
    print('PASS Tab reveals the workspace row, Space activates it, and bar controls fit the output')
    tab_to(driver, 'systemDetails')
    driver.xdo('key', 'space')
    driver.wait_controls('systemDetailsContent')
    driver.xdo('key', 'space')
    driver.wait_controls('systemDetailsContent', False)
    print('PASS keyboard disclosure expands and collapses extended system readings')
    print('INTERACTION PASS: desktop controls and confirmation flow')
