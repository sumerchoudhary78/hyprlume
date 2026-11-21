"""Only drive synthetic lock buttons; this check never locks a real display."""
def run(driver):
    driver.open_quick_settings()
    driver.wait_controls('lockSession');driver.focus_window();driver.click('lockSession')
    assert driver.request('lock')=={'op':'lock'}
    driver.click('openPower');driver.request('panel');driver.wait_controls('powerLock');driver.click('powerLock')
    assert driver.request('lock')=={'op':'lock'}
    print('LOCK CONTROLS PASS: bar and power controls request locking through the synthetic backend')
