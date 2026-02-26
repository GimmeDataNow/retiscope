// components/Sidebar.jsx
import "./Connection.css"

export function ConnectionLayout(props) {
  return (
    <div class="connection-layout-wrapper">
      <div class="connection-layout-wrapper-cosmetic">

        <div class="connections">
          <div class="connections-item">
            Local Server
          </div>
          <div class="connections-item">
            Remote Alma
          </div>
          <div class="connections-item">
            Remote Urath
          </div>
          <div class="connections-item">
            +
          </div>

        </div>

          {props.children} 
      </div>
    </div>
  );
}

