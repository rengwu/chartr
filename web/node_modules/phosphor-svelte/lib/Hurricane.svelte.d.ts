import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HurricaneIcon` instead.
 *
 * @example
 * ```svelte
 * <Hurricane color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Hurricane: Component<IconComponentProps, {}, "">;
type Hurricane = ReturnType<typeof Hurricane>;
export default Hurricane;
