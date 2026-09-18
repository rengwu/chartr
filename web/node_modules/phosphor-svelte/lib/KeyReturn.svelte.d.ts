import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `KeyReturnIcon` instead.
 *
 * @example
 * ```svelte
 * <KeyReturn color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const KeyReturn: Component<IconComponentProps, {}, "">;
type KeyReturn = ReturnType<typeof KeyReturn>;
export default KeyReturn;
