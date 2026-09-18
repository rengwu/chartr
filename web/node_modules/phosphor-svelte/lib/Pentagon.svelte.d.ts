import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PentagonIcon` instead.
 *
 * @example
 * ```svelte
 * <Pentagon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Pentagon: Component<IconComponentProps, {}, "">;
type Pentagon = ReturnType<typeof Pentagon>;
export default Pentagon;
