import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `AvocadoIcon` instead.
 *
 * @example
 * ```svelte
 * <Avocado color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Avocado: Component<IconComponentProps, {}, "">;
type Avocado = ReturnType<typeof Avocado>;
export default Avocado;
