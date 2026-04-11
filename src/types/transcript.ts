export interface Segment {
  id:              number;
  session_id:      number;
  speaker_id:      number;
  speaker_label:   string;
  display_name:    string | null;
  start_ms:        number;
  end_ms:          number;
  transcript_text: string;
}

export interface SpeakerNotification {
  id:              number;
  speech_swift_id: number;
  display_name:    string | null;
}
