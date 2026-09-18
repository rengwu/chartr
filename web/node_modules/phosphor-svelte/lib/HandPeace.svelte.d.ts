import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HandPeaceIcon` instead.
 *
 * @example
 * ```svelte
 * <HandPeace color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HandPeace: Component<IconComponentProps, {}, "">;
type HandPeace = ReturnType<typeof HandPeace>;
export default HandPeace;
