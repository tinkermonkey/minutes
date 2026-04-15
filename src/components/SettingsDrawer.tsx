import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import { Drawer, DrawerHeader, DrawerItems, Label, Select, TextInput, ToggleSwitch } from 'flowbite-react';
import type { AudioDevice } from '../types/device';

type VadMode = 'Silero' | 'WebRtc';

interface Props {
  isOpen: boolean;
  onClose: () => void;
}

export function SettingsDrawer({ isOpen, onClose }: Props) {
  const queryClient = useQueryClient();

  const { data: devices = [] } = useQuery<AudioDevice[]>({
    queryKey: ['audio_devices'],
    queryFn: (): Promise<AudioDevice[]> => invoke('get_audio_devices'),
  });

  // The user's saved preference — null/undefined means "use system default".
  const { data: preferredDevice } = useQuery<string | null>({
    queryKey: ['preferred_device'],
    queryFn: (): Promise<string | null> => invoke('get_preferred_device'),
  });

  const { data: port = '8080' } = useQuery<string>({
    queryKey: ['speech_swift_port'],
    queryFn: async (): Promise<string> => {
      const p = await invoke<string | null>('get_speech_swift_port');
      return p ?? '8080';
    },
  });

  const setDevice = useMutation({
    mutationFn: (deviceName: string): Promise<void> =>
      invoke('set_audio_device', { deviceName }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['preferred_device'] });
    },
  });

  const setPort = useMutation({
    mutationFn: (p: number): Promise<void> =>
      invoke('set_speech_swift_port', { port: p }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['speech_swift_port'] });
    },
  });

  const { data: vadMode, isLoading: vadModeLoading } = useQuery<VadMode>({
    queryKey: ['vad_mode'],
    queryFn: (): Promise<VadMode> => invoke<string>('get_vad_mode') as Promise<VadMode>,
  });

  const setVadMode = useMutation({
    mutationFn: (mode: VadMode): Promise<void> =>
      invoke('set_vad_mode', { mode }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vad_mode'] });
    },
  });

  // The currently active selection: user preference, or fall back to the
  // system default, or the first device in the list.
  const defaultDevice = devices.find(d => d.is_default);
  const selectedValue =
    preferredDevice ??
    defaultDevice?.name ??
    devices[0]?.name ??
    '';

  function handlePortBlur(value: string) {
    const parsed = Number(value);
    if (!Number.isNaN(parsed) && parsed >= 1024 && parsed <= 65535) {
      setPort.mutate(parsed);
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
              value={selectedValue}
              onChange={e => setDevice.mutate(e.target.value)}
            >
              {devices.map(device => (
                <option key={device.name} value={device.name}>
                  {device.name}{device.is_default ? ' (system default)' : ''}
                </option>
              ))}
            </Select>
            <p className="text-xs text-gray-500">
              Takes effect on the next recording session.
            </p>
          </div>

          <div className="flex flex-col gap-2">
            <Label>VAD Mode</Label>
            <ToggleSwitch
              checked={vadMode === 'Silero'}
              disabled={vadModeLoading || setVadMode.isPending}
              label={vadMode === 'Silero' ? 'Silero' : 'WebRTC'}
              onChange={(checked) => setVadMode.mutate(checked ? 'Silero' : 'WebRtc')}
            />
            <p className="text-xs text-gray-500">
              Silero is more accurate; WebRTC is lighter weight and works without the bundled model.
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
              onChange={e => {
                // Optimistic local update so the field feels responsive.
                queryClient.setQueryData(['speech_swift_port'], e.target.value);
              }}
              onBlur={e => handlePortBlur(e.target.value)}
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
