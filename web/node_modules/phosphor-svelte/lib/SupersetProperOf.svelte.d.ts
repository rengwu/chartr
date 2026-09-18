import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SupersetProperOfIcon` instead.
 *
 * @example
 * ```svelte
 * <SupersetProperOf color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SupersetProperOf: Component<IconComponentProps, {}, "">;
type SupersetProperOf = ReturnType<typeof SupersetProperOf>;
export default SupersetProperOf;
