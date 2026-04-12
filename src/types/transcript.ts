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

export interface ChunkSentEvent {
  start_ms:   number;
  end_ms:     number;
  sent_at_ms: number;
}

export interface ChunkProcessedEvent {
  start_ms:      number;
  response_ms:   number;
  word_count:    number;
  speaker_count: number;
}

/** Combined view of one chunk's send + response, as shown in the pipeline log. */
export interface PipelineEntry {
  start_ms:      number;
  end_ms:        number;
  sent_at_ms:    number;
  response_ms?:  number;
  word_count?:   number;
  speaker_count?: number;
}
