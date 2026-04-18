export interface Speaker {
  id:              number;
  speech_swift_id: number;
  display_name:    string | null;
  notes:           string | null;
  first_seen_at:   number;   // unix ms
  last_seen_at:    number;   // unix ms
  session_count:   number;
}

export interface SpeakerRenamedEvent {
  speech_swift_id: number;
  display_name:    string;
}

export interface SpeakersMergedEvent {
  src_id:           number;
  dst_id:           number;
  dst_display_name: string | null;
}

export interface SpeakerDeletedEvent {
  speech_swift_id: number;
}

export interface SpeakerRegistryResetEvent {}
