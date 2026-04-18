export interface Segment {
  id:              number;
  session_id:      number;
  speaker_id:      number | null;
  speaker_label:   string | null;
  display_name:    string | null;
  start_ms:        number;
  end_ms:          number;
  transcript_text: string;
  status:          'pending' | 'confirmed';
}

export interface SegmentsReplacedEvent {
  removed_ids: number[];
  added:       Segment[];
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
  best_score?:   number | null;
}

/** Fast-path entry: one short VAD chunk sent for immediate diarization. */
export interface FastPathEntry {
  kind:           'fast';
  start_ms:       number;
  end_ms:         number;
  sent_at_ms:     number;
  response_ms?:   number;
  word_count?:    number;
  speaker_count?: number;
  best_score?:    number | null;
}

/** Slow-path entry: a long accumulator clip sent for accurate diarization. */
export interface SlowPathEntry {
  kind:             'slow';
  start_ms:         number;
  end_ms:           number;
  clip_speech_secs: number;
  sent_at_ms:       number;
  response_ms?:     number;
  segment_count?:   number;
  best_score?:      number;
}

export type PipelineEntry = FastPathEntry | SlowPathEntry;

export interface AccumulatorUpdatedEvent {
  speech_secs:  number;
  trigger_secs: number;
}

export interface FastAccumulatorUpdatedEvent {
  speech_secs:  number;
  trigger_secs: number;
}

export interface SlowPathSentEvent {
  start_ms:         number;
  end_ms:           number;
  clip_speech_secs: number;
  sent_at_ms:       number;
}

export interface SlowPathDoneEvent {
  start_ms:      number;
  response_ms:   number;
  segment_count: number;
  best_score?:   number;
}
