const React = require('react');

class Footer extends React.Component {
  docUrl(doc, language) {
    const baseUrl = this.props.config.baseUrl;
    const lang = language && language !== 'en' ? language : '';
    return `${baseUrl}docs/${lang}${doc}`;
  }

  render() {
    return (
      <footer className="nav-footer" id="footer">
        <section className="sitemap" style={{textAlign: 'center'}}>
          <div>
            <h5>Docs</h5>
            <a href={this.docUrl('quick-start', this.props.language)}>
              Quick Start
            </a>
            <a href={this.docUrl('features', this.props.language)}>
              Features
            </a>
            <a href={this.docUrl('api', this.props.language)}>
              API Reference
            </a>
          </div>
          <div>
            <h5>Community</h5>
            <a href="https://github.com/replicante-io">GitHub Organisation</a>
            <a href="https://www.replicante.io/">Official Website</a>
          </div>
        </section>

        <section className="copyright">{this.props.config.copyright}</section>
      </footer>
    );
  }
}

module.exports = Footer;
