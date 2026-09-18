import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimpleMinusIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimpleMinus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimpleMinus: Component<IconComponentProps, {}, "">;
type FolderSimpleMinus = ReturnType<typeof FolderSimpleMinus>;
export default FolderSimpleMinus;
