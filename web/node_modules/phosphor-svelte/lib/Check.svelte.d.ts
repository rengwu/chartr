import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CheckIcon` instead.
 *
 * @example
 * ```svelte
 * <Check color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Check: Component<IconComponentProps, {}, "">;
type Check = ReturnType<typeof Check>;
export default Check;
