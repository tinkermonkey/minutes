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

export interface SpeakerSession {
  id:            number;
  created_at:    number;   // unix ms
  label:         string | null;
  duration_ms:   number | null;
  segment_count: number;
}

export interface SpeakerSegment {
  id:              number;
  session_id:      number;
  start_ms:        number;
  end_ms:          number;
  transcript_text: string;
  session_label:   string | null;
}

export interface SpeakerDetail {
  recent_sessions: SpeakerSession[];
  recent_segments: SpeakerSegment[];
}

export interface SimilarSpeaker {
  speaker:          Speaker;
  similarity_score: number;
}
