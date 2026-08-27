import { mount } from "svelte";
import "../../src/app.css";
import FixtureApp from "./FixtureApp.svelte";

mount(FixtureApp, { target: document.getElementById("app")! });
