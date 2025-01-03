/**
 * Represents a pseudo-probability score for an amino acid at a specific position
 */
export interface PseudoProbability {
  /** Position in the amino acid sequence (0-based index) */
  position: number;
  /** The amino acid character at this position */
  amino_acid: string;
  /** Logit score representing the pseudo-probability */
  pseudo_prob: number;
}
