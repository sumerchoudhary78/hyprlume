"""Clipboard/capture production controls using synthetic data and real input."""
def run(driver):
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
    driver.click('openDesktopTools');driver.request('panel');driver.wait_controls('toolsCapture');driver.click('toolsCapture');driver.request('panel')
    driver.wait_controls('recordScreen');driver.click('recordScreen')
    assert driver.request('capture')['request']=={'action':'record','region':False}
    driver.wait_controls('stopRecording');driver.click('stopRecording')
    assert driver.request('capture')['request']=={'action':'stop'}
    print('TOOLS INTERACTION PASS: opt-in, copy/remove/clear, screenshot selection and persistent recording stop')
