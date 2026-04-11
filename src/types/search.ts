export interface SearchResult {
  segment_id:          number;
  transcript_text:     string;
  start_ms:            number;
  end_ms:              number;
  speaker_id:          number | null;
  display_name:        string | null;
  session_id:          number;
  session_label:       string | null;
  session_created_at:  number;
  score:               number;
}

export interface SearchFilters {
  speaker_id:  number | null;
  start_date:  number | null;
  end_date:    number | null;
  limit?:      number;
}
