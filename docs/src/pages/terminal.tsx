import Layout from "@theme/Layout";
import '@xterm/xterm/css/xterm.css';
import '@xterm/xterm/lib/xterm.js';
import * as example from 'example-code';
import { useEffect, useLayoutEffect, useRef } from "react";


export default function Page(): JSX.Element {
  
   useEffect(() => {
      example.load();
   },[]);
    return (
      <Layout
        title={`Hello`}
        description="Description will go into a meta tag in <head />">
        <div id="terminal"></div>
      </Layout>
    );
  }
  