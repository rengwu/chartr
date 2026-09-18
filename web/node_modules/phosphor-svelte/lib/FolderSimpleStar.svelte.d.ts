import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FolderSimpleStarIcon` instead.
 *
 * @example
 * ```svelte
 * <FolderSimpleStar color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FolderSimpleStar: Component<IconComponentProps, {}, "">;
type FolderSimpleStar = ReturnType<typeof FolderSimpleStar>;
export default FolderSimpleStar;
