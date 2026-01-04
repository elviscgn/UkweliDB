# UkweliDB
A tamper proof, verifiable, immutable database, written from scratch in Rust.


<!-- <details> -->
  <!-- <summary>How it works</summary> -->
<h2>Core Idea </h2>
<p>In traditional databases, records can be edited after creation and without leaving a clear trail. For example, someone could change the outcome of a government procurement bid or a financial transaction. This is a problem because auditors reviewing the database later on will have no reliable way to determine whether data has been altered and is trustworthy.</p>

<h3>Records </h3>
<img src="assets/records.png"/>
<p>To address this problem, we organize records into a cryptographically linked chain (taking heavy inspiration from how blockchain ensures immutability). Starting with the genesis record (the first record that everything starts from). Each record stores its data (the payload) and a hash of that payload and the hash of the previous record in the chain. This way the hash acts like a digital fingerprint and if you tried to change the payload of any record it would change the hash which in return will break the chain revealing that it was tampered with.  </p>

<h3>Modified Record example </h3>
<img src="assets/modified_record.png"/>
when verifying, this will immediately break and show us where the data was changed.
<!-- </select> -->