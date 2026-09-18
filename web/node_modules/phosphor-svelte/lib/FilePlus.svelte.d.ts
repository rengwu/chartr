import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FilePlusIcon` instead.
 *
 * @example
 * ```svelte
 * <FilePlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FilePlus: Component<IconComponentProps, {}, "">;
type FilePlus = ReturnType<typeof FilePlus>;
export default FilePlus;
