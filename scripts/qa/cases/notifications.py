"""Real pointer/keyboard checks against synthetic notification state only."""


def run(driver):
    driver.wait_controls('openNotifications')
    driver.focus_window()
    driver.click('openNotifications')
    assert driver.request('notifications')['request']['action'] == 'toggle'
    driver.wait_controls('notificationDnd')
    driver.click('notificationDnd')
    request = driver.request('notifications')['request']
    assert request == {'action': 'dnd', 'enabled': True}, request
    driver.wait_event(lambda event: event['kind'] == 'controls')
    driver.click('noticeAction1_0')
    request = driver.request('notifications')['request']
    assert request == {'action': 'invoke', 'id': 1, 'revision': 1, 'key': 'default'}, request
    driver.wait_controls('noticeAction1_0', False)
    driver.click('removeNotice1')
    request = driver.request('notifications')['request']
    assert request == {'action': 'remove', 'id': 1, 'revision': 1}, request
    driver.wait_controls('removeNotice1', False)
    print('PASS DND, revision-bound actions and history removal use the service topic')
    driver.click('clearNotifications')
    assert driver.request('notifications')['request']['action'] == 'clear'
    driver.wait_event(lambda event: event['kind'] == 'controls' and not event['controls'].get('clearNotifications', {}).get('enabled', True))
    driver.xdo('key', 'Escape')
    assert driver.request('notifications')['request']['action'] == 'close'
    driver.wait_controls('notificationDnd', False)
    print('NOTIFICATIONS INTERACTION PASS: centre, DND, actions, removal, clear and Escape')
