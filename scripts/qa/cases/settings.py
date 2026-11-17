"""Edit settings through the production QML without touching real configuration."""
def run(driver):
    driver.wait_controls('openSettings');driver.focus_window();driver.click('openSettings')
    assert driver.request('panel')['panel']=='settings'
    driver.wait_controls('settingsReduced');driver.click('settingsReduced')
    driver.click('settingsAccentviolet')
    driver.click('settingsScale');driver.xdo('key','ctrl+a');driver.xdo('type','--clearmodifiers','1.05')
    driver.wait_event(lambda event:event['kind']=='controls' and event['controls'].get('settingsSave',{}).get('enabled',False))
    driver.click('settingsSave')
    request=driver.request('settings')['request'];assert request['action']=='save' and request['revision']==1
    appearance=request['values']['preferences']['appearance']
    assert appearance['reduced_effects'] and appearance['accent']=='violet' and appearance['scale']==1.05,appearance
    driver.wait_event(lambda event:event['kind']=='controls' and not event['controls'].get('settingsSave',{}).get('enabled',True))
    driver.click('settingsTab3');driver.wait_controls('settingsDndStartup');driver.click('settingsDndStartup')
    driver.wait_event(lambda event:event['kind']=='controls' and event['controls'].get('settingsSave',{}).get('enabled',False))
    driver.click('settingsSave')
    assert driver.request('settings')['request']['values']['preferences']['services']['notifications_dnd']
    driver.wait_event(lambda event:event['kind']=='controls' and not event['controls'].get('settingsSave',{}).get('enabled',True))
    driver.click('settingsTab2');driver.wait_controls('settingsKeyLauncher');driver.click('settingsKeyLauncher');driver.xdo('key','ctrl+a');driver.xdo('type','--clearmodifiers','SUPER+F2')
    driver.wait_event(lambda event:event['kind']=='controls' and event['controls'].get('settingsSave',{}).get('enabled',False))
    driver.click('settingsSave');assert driver.request('settings')['request']['values']['preferences']['shortcuts']['launcher']=='SUPER+F2'
    driver.wait_event(lambda event:event['kind']=='controls' and not event['controls'].get('settingsSave',{}).get('enabled',True))
    driver.click('settingsTab4');driver.wait_controls('settingsIdleEnabled');driver.click('settingsIdleEnabled')
    driver.wait_event(lambda event:event['kind']=='controls' and event['controls'].get('settingsSave',{}).get('enabled',False))
    driver.click('settingsSave')
    idle=driver.request('settings')['request']['values']['preferences']['idle']
    assert idle['enabled'] and idle['suspend_after_seconds']==0 and idle['dim_percent']==20,idle
    driver.wait_event(lambda event:event['kind']=='controls' and not event['controls'].get('settingsSave',{}).get('enabled',True))
    driver.click('settingsReload');assert driver.request('settings')['request']['action']=='reload'
    driver.click('closeSettings');assert driver.request('panel')['open'] is False
    print('SETTINGS INTERACTION PASS: reduced glass, accent, decimal scale, startup defaults, shortcuts, save acknowledgments and reload')
