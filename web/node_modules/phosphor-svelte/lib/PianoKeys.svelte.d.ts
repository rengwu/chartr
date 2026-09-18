import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PianoKeysIcon` instead.
 *
 * @example
 * ```svelte
 * <PianoKeys color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PianoKeys: Component<IconComponentProps, {}, "">;
type PianoKeys = ReturnType<typeof PianoKeys>;
export default PianoKeys;
