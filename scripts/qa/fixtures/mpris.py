"""Small MPRIS test player. The runner supplies a private D-Bus address."""
import asyncio

from dbus_next import Variant
from dbus_next.aio import MessageBus
from dbus_next.constants import PropertyAccess
from dbus_next.service import ServiceInterface, dbus_property, method


class Identity(ServiceInterface):
    def __init__(self):
        super().__init__('org.mpris.MediaPlayer2')

    @dbus_property(access=PropertyAccess.READ)
    def Identity(self) -> 's':
        return 'Hyprlume QA'

    @dbus_property(access=PropertyAccess.READ)
    def DesktopEntry(self) -> 's':
        return 'hyprlume-qa'

    @dbus_property(access=PropertyAccess.READ)
    def CanQuit(self) -> 'b':
        return False

    @dbus_property(access=PropertyAccess.READ)
    def CanRaise(self) -> 'b':
        return False

    @dbus_property(access=PropertyAccess.READ)
    def HasTrackList(self) -> 'b':
        return False

    @dbus_property(access=PropertyAccess.READ)
    def SupportedUriSchemes(self) -> 'as':
        return []

    @dbus_property(access=PropertyAccess.READ)
    def SupportedMimeTypes(self) -> 'as':
        return []


class Player(ServiceInterface):
    def __init__(self):
        super().__init__('org.mpris.MediaPlayer2.Player')
        self.playing = True
        self.title = 'Fixture track'

    @dbus_property(access=PropertyAccess.READ)
    def PlaybackStatus(self) -> 's':
        return 'Playing' if self.playing else 'Paused'

    @dbus_property(access=PropertyAccess.READ)
    def Metadata(self) -> 'a{sv}':
        return {'mpris:trackid': Variant('o', '/hyprlume/qa/track'),
                'xesam:title': Variant('s', self.title),
                'xesam:artist': Variant('as', ['Fixture artist'])}

    @dbus_property(access=PropertyAccess.READ)
    def Rate(self) -> 'd':
        return 1.0

    @dbus_property(access=PropertyAccess.READ)
    def MinimumRate(self) -> 'd':
        return 1.0

    @dbus_property(access=PropertyAccess.READ)
    def MaximumRate(self) -> 'd':
        return 1.0

    @dbus_property(access=PropertyAccess.READ)
    def Position(self) -> 'x':
        return 0

    @dbus_property(access=PropertyAccess.READ)
    def CanControl(self) -> 'b':
        return True

    @dbus_property(access=PropertyAccess.READ)
    def CanPlay(self) -> 'b':
        return True

    @dbus_property(access=PropertyAccess.READ)
    def CanPause(self) -> 'b':
        return True

    @dbus_property(access=PropertyAccess.READ)
    def CanGoNext(self) -> 'b':
        return True

    @dbus_property(access=PropertyAccess.READ)
    def CanGoPrevious(self) -> 'b':
        return False

    @dbus_property(access=PropertyAccess.READ)
    def CanSeek(self) -> 'b':
        return False

    @method()
    def PlayPause(self):
        self.playing = not self.playing
        self.emit_properties_changed({'PlaybackStatus': self.PlaybackStatus})

    @method()
    def Play(self):
        self.playing = True
        self.emit_properties_changed({'PlaybackStatus': self.PlaybackStatus})

    @method()
    def Pause(self):
        self.playing = False
        self.emit_properties_changed({'PlaybackStatus': self.PlaybackStatus})

    @method()
    def Next(self):
        self.title = 'Next fixture track'
        self.emit_properties_changed({'Metadata': self.Metadata})


async def main():
    bus = await MessageBus().connect()
    bus.export('/org/mpris/MediaPlayer2', Identity())
    bus.export('/org/mpris/MediaPlayer2', Player())
    await bus.request_name('org.mpris.MediaPlayer2.hyprlume_qa')
    print('MPRIS_READY', flush=True)
    await asyncio.Future()


if __name__ == '__main__':
    asyncio.run(main())
