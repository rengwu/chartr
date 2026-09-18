import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LampPendantIcon` instead.
 *
 * @example
 * ```svelte
 * <LampPendant color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LampPendant: Component<IconComponentProps, {}, "">;
type LampPendant = ReturnType<typeof LampPendant>;
export default LampPendant;
