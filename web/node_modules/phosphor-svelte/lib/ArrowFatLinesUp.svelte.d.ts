import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowFatLinesUpIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowFatLinesUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowFatLinesUp: Component<IconComponentProps, {}, "">;
type ArrowFatLinesUp = ReturnType<typeof ArrowFatLinesUp>;
export default ArrowFatLinesUp;
