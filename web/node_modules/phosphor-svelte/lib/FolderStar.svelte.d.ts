import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderStarIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderStar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderStar: Component<IconComponentProps, {}, "">;
type FolderStar = ReturnType<typeof FolderStar>;
export default FolderStar;
