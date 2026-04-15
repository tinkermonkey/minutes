export interface SessionParticipant {
  speech_swift_id: number;
  display_name:    string | null;
}

export interface Session {
  id:           number;
  created_at:   number;
  label:        string | null;
  duration_ms:  number | null;
  source:       string;
  participants: SessionParticipant[];
}

export interface SessionsPage {
  sessions:    Session[];
  total_count: number;
}

export interface SegmentWithSpeaker {
  id:              number;
  session_id:      number;
  speaker_id:      number | null;
  start_ms:        number;
  end_ms:          number;
  transcript_text: string;
  display_name:    string | null;
  status:          'pending' | 'confirmed';
}

export type SortBy  = 'date' | 'duration';
export type SortDir = 'asc' | 'desc';

export interface SessionFilter {
  start_date: number | null;
  end_date:   number | null;
  sort_by:    SortBy;
  sort_dir:   SortDir;
  page:       number;
  page_size:  number;
}
