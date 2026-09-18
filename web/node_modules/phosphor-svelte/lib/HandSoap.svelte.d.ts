import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandSoapIcon` instead.
 *
 * @example
 * ```svelte
 * <HandSoap color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandSoap: Component<IconComponentProps, {}, "">;
type HandSoap = ReturnType<typeof HandSoap>;
export default HandSoap;
