import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { Drawer, DrawerHeader, DrawerItems, Label, Select, TextInput } from 'flowbite-react';
import type { AudioDevice } from '../types/device';

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDrawer({ isOpen, onClose }: Props) {
  const queryClient = useQueryClient();
  const [port, setPort] = useState('8080');

  const { data: devices = [] } = useQuery<AudioDevice[]>({
    queryKey: ['audio_devices'],
    queryFn: (): Promise<AudioDevice[]> => invoke('get_audio_devices'),
  });

  const setDevice = useMutation({
    mutationFn: (deviceName: string): Promise<void> =>
      invoke('set_audio_device', { deviceName }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['audio_devices'] });
    },
  });

  const defaultDevice = devices.find(d => d.is_default);

  function handlePortBlur() {
    const parsed = Number(port);
    if (!Number.isNaN(parsed) && parsed >= 1024 && parsed <= 65535) {
      invoke('set_speech_swift_port', { port: parsed });
    }
  }

  return (
    <Drawer open={isOpen} onClose={onClose} position="right" className="w-80">
      <DrawerHeader title="Settings" />
      <DrawerItems>
        <div className="flex flex-col gap-4 p-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="audio-device">Audio Input Device</Label>
            <Select
              id="audio-device"
              value={defaultDevice?.name ?? ''}
              onChange={e => setDevice.mutate(e.target.value)}
            >
              {devices.map(device => (
                <option key={device.name} value={device.name}>
                  {device.name}{device.is_default ? ' (default)' : ''}
                </option>
              ))}
            </Select>
            <p className="text-xs text-gray-500">
              Takes effect on the next recording session.
            </p>
          </div>

          <div className="flex flex-col gap-2">
            <Label htmlFor="swift-port">speech-swift Port</Label>
            <TextInput
              id="swift-port"
              type="number"
              min={1024}
              max={65535}
              value={port}
              onChange={e => setPort(e.target.value)}
              onBlur={handlePortBlur}
            />
            <p className="text-xs text-gray-500">
              Default: 8080. Takes effect on next app launch.
            </p>
          </div>
        </div>
      </DrawerItems>
    </Drawer>
  );
}
