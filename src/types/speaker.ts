export interface Speaker {
  id:              number;
  speech_swift_id: number;
  display_name:    string | null;
  notes:           string | null;
  first_seen_at:   number;   // unix ms
  last_seen_at:    number;   // unix ms
  session_count:   number;
}
