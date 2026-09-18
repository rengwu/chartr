import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PuzzlePieceIcon` instead.
 *
 * @example
 * ```svelte
 * <PuzzlePiece color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PuzzlePiece: Component<IconComponentProps, {}, "">;
type PuzzlePiece = ReturnType<typeof PuzzlePiece>;
export default PuzzlePiece;
