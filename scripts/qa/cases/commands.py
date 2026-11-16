"""Agent command, review and dashboard controls."""


def run(driver, reduced_effects=False):
    driver.wait_controls('quickCommand')
    driver.focus_window()
    driver.click('quickCommand'); driver.request('toggle'); driver.wait_controls('commandInput')
    driver.xdo('type', '--clearmodifiers', '--delay', '25', 'workspace 2'); driver.xdo('key', 'Return')
    r = driver.request('command')
    assert r['text']=='workspace 2' and r['dry_run'] is True, r
    driver.wait_event(lambda e:e['kind']=='controls' and 'commandInput' in e['controls'] and not e['controls']['commandInput']['enabled'] and 'confirmPlan' not in e['controls'])
    assert not driver.controls['previewCommand']['enabled'] and not driver.controls['runDirect']['enabled'], 'busy controls remained enabled'
    driver.wait_controls('confirmPlan')
    assert not driver.controls['commandInput']['enabled'], 'review allowed a replacement command'
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
    driver.click('workspaceRow2'); e=driver.wait_event(lambda e:e['kind']=='workspace'); assert e['name']=='Operations',e
    driver.click('mode4'); e=driver.wait_event(lambda e:e['kind']=='mode'); assert e['mode']==4,e
    print('PASS transformed workspace rows and top-bar navigation receive real pointer input')
    if not reduced_effects:
        driver.click('hologramGlobeRegion'); e=driver.wait_event(lambda e:e['kind']=='mode'); assert e['mode']==5,e
        driver.click('hologramCityRegion'); e=driver.wait_event(lambda e:e['kind']=='mode'); assert e['mode']==4,e
        print('PASS real 3D globe and city route to network and container views')
    else:
        assert 'hologramGlobeRegion' not in driver.controls and 'hologramCityRegion' not in driver.controls
    driver.click('agentVolume'); r=driver.request('command'); assert r['text']=='volume 35' and r['dry_run'] is True,r
    driver.wait_controls('cancelPlan'); driver.click('cancelPlan'); driver.request('cancel')
    driver.xdo('key','Escape'); driver.request('toggle'); driver.wait_controls('commandInput',False)
    driver.click('launcherterminal'); r=driver.request('command'); assert r['text']=='launch terminal' and r['dry_run'] is True,r
    print('PASS agent shortcut previews; dismissal cancels; spatial terminal launcher previews an allowlisted app')
    print('INTERACTION PASS: desktop controls and confirmation flow')
