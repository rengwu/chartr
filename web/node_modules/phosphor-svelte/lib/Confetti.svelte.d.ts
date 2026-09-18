import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ConfettiIcon` instead.
 *
 * @example
 * ```svelte
 * <Confetti color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Confetti: Component<IconComponentProps, {}, "">;
type Confetti = ReturnType<typeof Confetti>;
export default Confetti;
