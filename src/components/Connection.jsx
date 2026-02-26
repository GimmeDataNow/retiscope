// components/Sidebar.jsx
import "./Connection.css"

export function Connection(props) {
  return (
    <div class="connection">
      
      <div class="connection-settings">
        Server
        <div class="connection-widgets">
          <div class="connection-widget-info">
            status:
            <div class="server-state">OK</div>
          </div>
          <div class="connection-widget-info">
            connection:
            <div class="server-state">OK</div>
          </div>
        </div>
        <div class="connection-widgets">
          <div class="connection-widget-info">
            Interface type:
            <div class="server-state">HTTP/HTTPS</div>
          </div>
          <div class="connection-widget-info">
            Address:
            <div class="server-state">127.0.0.1:8888</div>
          </div>
        </div>
        <div class="connection-control-arguments">
          Arguments will be listed here.
        </div>

        <div class="connection-controls">
          <button class="connection-buttons">Start</button>
          <button class="connection-buttons">Restart</button>
          <button class="connection-buttons">Disconnect</button>
          <button class="connection-buttons">Delete</button>
        </div>
      </div>

      <div class="connection-logs" style={{ flex: 1 }}>
        <div class="connection-logs-bar">
          <div>Logs</div>
          <div>paused: false</div>
          <div>Filter here</div>
        </div>
        {props.children} 
      </div>
    </div>
  );
}

