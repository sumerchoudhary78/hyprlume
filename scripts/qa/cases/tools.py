"""Clipboard/capture production controls using synthetic data and real input."""
def run(driver):
    driver.open_quick_settings()
    driver.wait_controls('openDesktopTools');driver.focus_window();driver.click('openDesktopTools')
    assert driver.request('panel')['panel']=='clipboard'
    driver.wait_controls('clipboardEnable');driver.click('clipboardEnable')
    assert driver.request('clipboard')['request']=={'action':'enable','enabled':True}
    driver.click('clipboardCopy0');assert driver.request('clipboard')['request']=={'action':'copy','id':1}
    driver.click('clipboardRemove0');assert driver.request('clipboard')['request']=={'action':'remove','id':1}
    driver.click('clipboardClear');assert driver.request('clipboard')['request']=={'action':'clear'}
    driver.wait_controls('clipboardCopy0',False)
    driver.click('toolsCapture');assert driver.request('panel')['panel']=='capture'
    driver.wait_controls('screenshotRegion');driver.click('screenshotRegion')
    assert driver.request('capture')['request']=={'action':'screenshot','region':True}
    driver.wait_controls('closeTools',False)
    driver.open_quick_settings()
    driver.click('openDesktopTools');driver.request('panel');driver.wait_controls('toolsCapture');driver.click('toolsCapture');driver.request('panel')
    driver.wait_controls('recordScreen');driver.click('recordScreen')
    assert driver.request('capture')['request']=={'action':'record','region':False,'output':'PREVIEW-1'}
    driver.wait_controls('stopRecording')
    assert driver.controls['quickSettings']['inside'], "Recording control clips Quick settings"
    driver.click('stopRecording')
    assert driver.request('capture')['request']=={'action':'stop'}
    driver.wait_controls('stopRecording', False)
    driver.open_quick_settings()
    driver.wait_stable_controls('openDesktopTools')
    driver.click('openDesktopTools')
    driver.request('panel')
    driver.wait_controls('toolsDrives')
    driver.click('toolsDrives')
    assert driver.request('panel')['panel'] == 'storage'
    driver.wait_controls('driveUnlock0_1')
    driver.click('driveUnlock0_1')
    driver.wait_controls('drivePassphrase')
    driver.click('drivePassphrase')
    driver.xdo('type', '--clearmodifiers', 'fixture-passphrase')
    driver.wait_event(lambda event: event['kind'] == 'controls' and event['controls'].get('driveConfirm', {}).get('enabled', False))
    driver.click('driveConfirm')
    assert driver.request('storage')['request'] == {'action': 'unlock', 'id': 'fixture-encrypted', 'password': '[credential]'}
    driver.wait_controls('driveEject0')
    driver.click('driveEject0')
    driver.wait_controls('driveConfirm')
    driver.click('driveConfirm')
    assert driver.request('storage')['request'] == {'action': 'eject', 'id': 'fixture-drive'}
    print('TOOLS INTERACTION PASS: opt-in, copy/remove/clear, screenshot selection and persistent recording stop')
